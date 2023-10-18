package main

import (
	"context"
	"encoding/json"
	"fmt"
	"github.com/aws/aws-lambda-go/events"
	runtime "github.com/aws/aws-lambda-go/lambda"
	"github.com/aws/aws-lambda-go/lambdacontext"
	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/aws/aws-sdk-go/service/lambda"
	. "marcusb.org/golang/debug"
	"net/http"
	"net/url"
	"os"
	"strings"
)

var client = lambda.New(session.New())

var ENV_F3_REGION string
var ENV_SLACK_API_KEY string
var ENV_SLACK_INVITE_LINK string
var ENV_SLACK_SIGNING_SECRET string
var ENV_SLACK_CHANNEL_ID string
var ENV_EMAIL_SENDER_ADDRESS string
var ENV_MAILCHIMP_API_KEY string
var ENV_MAILCHIMP_API_ENDPOINT string
var ENV_MAILCHIMP_LIST_ID string

type Response struct {
	StatusCode int               `json:"statusCode"`
	Headers    map[string]string `json:"headers"`
	Body       string            `json:"body"`
}

type RequestBody struct {
	Data    any    `json:"data"`
	Message string `json:"message"`
}

func parse_args(input string) map[string]string {
	inputmap := make(map[string]string)

	input, _ = url.QueryUnescape(input)
	keyvals := strings.Split(input, "&")
	for _, key := range keyvals {
		splitkey := strings.Split(key, "=")
		inputmap[splitkey[0]] = splitkey[1]
	}

	return inputmap
}

func StartupLogs(ctx context.Context, event events.APIGatewayProxyRequest) {
	eventJson, _ := json.MarshalIndent(event, "", "  ")
	LogPrint(fmt.Sprintf("EVENT: %s", eventJson), LogLevelDebug)

	if len(string(event.Body)) > 0 {
		args := parse_args(event.Body)
		LogPrint("ALL FORM VARS:", LogLevelDebug)
		for key := range args {
			LogPrint(fmt.Sprintf("%s: %s", key, args[key]), LogLevelDebug)
		}
	}

	LogPrint(fmt.Sprintf("REGION: %s", os.Getenv("AWS_REGION")), LogLevelDebug)
	LogPrint("ALL ENV VARS:", LogLevelDebug)
	for _, element := range os.Environ() {
		LogPrint(element, LogLevelDebug)
	}

	lc, _ := lambdacontext.FromContext(ctx)
	LogPrint(fmt.Sprintf("REQUEST ID: %s", lc.AwsRequestID), LogLevelDebug)

	LogPrint(fmt.Sprintf("FUNCTION NAME: %s", lambdacontext.FunctionName), LogLevelDebug)

	deadline, _ := ctx.Deadline()
	LogPrint(fmt.Sprintf("DEADLINE: %s", deadline), LogLevelDebug)
}

func handleRequest(ctx context.Context, event events.APIGatewayProxyRequest) (Response, error) {
	LogInit()

	ENV_F3_REGION = os.Getenv("f3_region")
	ENV_SLACK_API_KEY = os.Getenv("slack_api_key")
	ENV_SLACK_INVITE_LINK = os.Getenv("slack_invite_link")
	ENV_SLACK_SIGNING_SECRET = os.Getenv("slack_signing_secret")
	ENV_SLACK_CHANNEL_ID = os.Getenv("slack_channel_id")
	ENV_EMAIL_SENDER_ADDRESS = os.Getenv("email_sender_address")
	ENV_MAILCHIMP_API_KEY = os.Getenv("mailchimp_api_key")
	ENV_MAILCHIMP_API_ENDPOINT = os.Getenv("mailchimp_api_endpoint")
	ENV_MAILCHIMP_LIST_ID = os.Getenv("mailchimp_list_id")

	StartupLogs(ctx, event)

	args := parse_args(event.Body)

	/* Each Slack message contains a HMAC-SHA 256 signature. */
	if validate_signature(event.Body, event.Headers["X-Slack-Signature"],
		event.Headers["X-Slack-Request-Timestamp"]) == false {
		LogPrint("INVALID Slack Message Signature", LogLevelInfo)
		return Response{}, nil
	}

	/* Slack uses the same URI for all app actions, so we need to use request fields to distinguish
	 * between different calls.  There are currently three inbound requests we handle:
	 * we need to use other aspects of the payload to differentiate between three calls...
	 *
	 * 1 - A request made via a slash action for the modal dialog will contain a trigger_id field in
	 * the POST data, but no payload field.
	 *
	 * 2 - A request made via a button on the app home page to launch the modal dialog will contain
	 * a payload field; the type will be set to block_actions and the trigger_id will be in the
	 * JSON payload.
	 *
	 * 3 - A request with user-submitted form data will have a payload field, which will contain
	 * the form data.
	 */
	if len(args["payload"]) > 0 {
		var payload_args map[string]any

		json.Unmarshal([]byte(args["payload"]), &payload_args)

		if payload_args["type"] == "view_submission" {
			LogPrint("Handling view_submission...", LogLevelDebug)
			return view_submission(ctx, event)
		} else if payload_args["type"] == "block_actions" {
			trigger_id, ok := payload_args["trigger_id"].(string)

			if ok == false {
				LogPrint("trigger_id in payload is not a string!", LogLevelDebug)
				return Response{}, nil
			}
			LogPrint("Handling Modal Display call...", LogLevelDebug)
			return view_display(trigger_id)
		}
		return Response{}, nil
	} else if len(args["trigger_id"]) > 0 {
		LogPrint("Handling Modal Display call...", LogLevelDebug)
		return view_display(args["trigger_id"])
	} else {
		LogPrint("Received unknown request...", LogLevelDebug)
		LogPrint(args["type"], LogLevelDebug)
		var data = json.RawMessage(fmt.Sprintf(`{"Status":"Unknown Request","error": "%q"}`,
			nil))
		response := Response{}
		response.StatusCode = http.StatusOK
		response.Body = string(data)
		return response, nil
	}
}

func main() {
	runtime.Start(handleRequest)
}
