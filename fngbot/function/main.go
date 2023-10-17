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
	ENV_EMAIL_SENDER_ADDRESS = os.Getenv("email_sender_address")
	ENV_MAILCHIMP_API_KEY = os.Getenv("mailchimp_api_key")
	ENV_MAILCHIMP_API_ENDPOINT = os.Getenv("mailchimp_api_endpoint")
	ENV_MAILCHIMP_LIST_ID = os.Getenv("mailchimp_list_id")

	StartupLogs(ctx, event)

	args := parse_args(event.Body)

	/* Slack uses the same URI to request the modal display and to return the submitted data, so
	 * we need to use other aspects of the payload to differentiate between the two calls...
	 * A request for the modal display will contain a trigger_id field in the POST data, but no
	 * payload field.
	 * A call with user-submitted data will have both a trigger_id and a payload field present.
	 */
	if len(args["payload"]) > 0 {
		var payload_args map[string]any

		json.Unmarshal([]byte(args["payload"]), &payload_args)

		if payload_args["type"] == "view_submission" {
			LogPrint("Handling view_submission...", LogLevelDebug)
			return view_submission(ctx, event)
		}
		return Response{}, nil
	} else if len(args["trigger_id"]) > 0 {
		LogPrint("Handling Modal Display call...", LogLevelDebug)
		return view_display(ctx, event)
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
