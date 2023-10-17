/*
 * slack.go -- Slack-related functions for the FNG bot.
 */

package main

import (
	"context"
	"encoding/json"
	"fmt"
	"github.com/aws/aws-lambda-go/events"
	"io"
	. "marcusb.org/golang/debug"
	"net/http"
	"strings"
)

type ViewSubmissionBlockLabel struct {
	Text string
}

type ViewSubmissionBlockElement struct {
	Action_Id string
}

type ViewSubmissionBlock struct {
	Label   ViewSubmissionBlockLabel
	Element ViewSubmissionBlockElement
}

type ViewSubmissionState struct {
	Values map[string]map[string]map[string]string
}

type ViewSubmissionView struct {
	Blocks []ViewSubmissionBlock
	State  ViewSubmissionState
}

type ViewSubmission struct {
	Type       string
	Token      string
	Trigger_id string
	View       ViewSubmissionView
}

var ModalPayload string = `
{
  "trigger_id": "%s",
  "view": {
        "title": {
                "type": "plain_text",
                "text": "FNG Bot",
                "emoji": true
        },
        "submit": {
                "type": "plain_text",
                "text": "Invite!",
                "emoji": true
        },
        "type": "modal",
        "blocks": [
                {
                        "type": "input",
                        "element": {
                                "type": "plain_text_input"
                        },
                        "label": {
                                "type": "plain_text",
                                "text": "F3 Name",
                                "emoji": false
                        }
                },
                {
                        "type": "input",
                        "element": {
                                "type": "plain_text_input"
                        },
                        "label": {
                                "type": "plain_text",
                                "text": "Hospital Name",
                                "emoji": false
                        }
                },
                {
                        "type": "input",
                        "element": {
                                "type": "email_text_input"
                        },
                        "label": {
                                "type": "plain_text",
                                "text": "Email Address",
                                "emoji": false
                        }
                },
                {
                        "type": "input",
                        "element": {
                                "type": "number_input",
                                "is_decimal_allowed": false,
                                "action_id": "number_input-action"
                        },
                        "label": {
                                "type": "plain_text",
                                "text": "Cell Phone",
                                "emoji": false
                        }
                }
        ]
}
}`

var ModalUpdatePayload string = `
{
  "response_action": "update",
  "view": {
    "type": "modal",
    "title": {
      "type": "plain_text",
      "text": "Status"
    },

    "blocks": [
               {
                "type": "section",
                "text": {
                        "type": "mrkdwn",
                        "text": "Thanks for using FNG Bot!\n*Status Report*:\n\n✅ I am a robot 🤖!\n\n %s\n\n%s\n\n"
                        }
                }
        ]
    }
}`

func view_display(ctx context.Context, event events.APIGatewayProxyRequest) (Response, error) {
	args := parse_args(event.Body)

	httpclient := &http.Client{}

	req, _ := http.NewRequest("POST", "https://slack.com/api/views.open",
		strings.NewReader(fmt.Sprintf(ModalPayload, args["trigger_id"])))
	req.Header.Add("Content-type", "application/json")
	req.Header.Add("Authorization", fmt.Sprintf("Bearer %s", ENV_SLACK_API_KEY))

	resp, err := httpclient.Do(req)

	body, _ := io.ReadAll(resp.Body)

	response := Response{}
	response.StatusCode = http.StatusOK

	if err != nil {
		var data = json.RawMessage(
			fmt.Sprintf(`{"Status":"Error invoking form!", "Response": "%q","error": "%q"}`,
				string(body), err))
		response.Body = string(data)
	}

	return response, err
}

func view_submission(ctx context.Context, event events.APIGatewayProxyRequest) (Response, error) {
	var view ViewSubmission

	LogPrint("Handling view_submission...", LogLevelDebug)
	args := parse_args(event.Body)

	json.Unmarshal([]byte(args["payload"]), &view)

	f3_name := ""
	hospital_name := ""
	email := ""
	cell := ""

	for _, val := range view.View.State.Values {
		for ikey, _ := range val {
			for _, bval := range view.View.Blocks {
				if bval.Element.Action_Id == ikey {
					switch bval.Label.Text {
					case "F3 Name":
						f3_name = val[ikey]["value"]
					case "Hospital Name":
						hospital_name = val[ikey]["value"]
					case "Email Address":
						email = val[ikey]["value"]
					case "Cell Phone":
						cell = val[ikey]["value"]
					}
				}
			}
		}
	}
	LogPrint(fmt.Sprintf("Found Modal Values %s, %s, %s, %s", f3_name, hospital_name, email, cell),
		LogLevelDebug)

	/* Add user to Mailchimp */
	mailchimp_status := ""
	switch mailchimp_add(f3_name, hospital_name, email, cell) {
	case MailChimpSuccess:
		mailchimp_status = "✅ Success adding user to the mailing list!"
	case MailChimpExists:
		mailchimp_status = "⚠️  Email address is already subscribed to the mailing list."
	case MailChimpFailure:
		mailchimp_status = "🛑 Could not add user to the mailing list."
	}

	/* Invite FNG to Slack via email */
	slack_invite_status := ""
	switch ses_send_invite(ENV_EMAIL_SENDER_ADDRESS, email, ENV_F3_REGION, ENV_SLACK_INVITE_LINK) {
	case EmailSuccess:
		slack_invite_status = "✅ Success inviting user to Slack!"
	case EmailTryAgain:
		slack_invite_status = "⚠️  I couldn't invite the user to Slack, but try again later."
	case EmailFailure:
		slack_invite_status = "🛑 I could not invite the user to Slack."
	}

	var data = json.RawMessage(fmt.Sprintf(ModalUpdatePayload, mailchimp_status, slack_invite_status))

	response := Response{}
	response.StatusCode = http.StatusOK
	response.Body = string(data)
	return response, nil
}
