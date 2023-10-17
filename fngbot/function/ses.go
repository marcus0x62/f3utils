/*
 * Email Invites using AWS Simple Email Service
 */
package main

import (
	"fmt"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/awserr"
	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/aws/aws-sdk-go/service/ses"
	. "marcusb.org/golang/debug"
)

const (
	EmailSuccess  = iota
	EmailTryAgain = iota
	EmailFailure  = iota
)

func ses_send_invite(from string, to string, region string, link string) int {
	svc := ses.New(session.New(&aws.Config{Region: aws.String("us-east-2")}))
	input := &ses.SendEmailInput{
		Destination: &ses.Destination{
			ToAddresses: []*string{
				aws.String(to),
			},
		},
		Message: &ses.Message{
			Body: &ses.Body{
				Html: &ses.Content{
					Charset: aws.String("UTF-8"),
					Data:    aws.String(fmt.Sprintf("Hi! Please join %s on Slack by clicking this <a href=\"%s\">link</a>.", region, link)),
				},
				Text: &ses.Content{
					Charset: aws.String("UTF-8"),
					Data:    aws.String(fmt.Sprintf("Hi! Please join %s on Slack by clicking this link: %s", region, link)),
				},
			},
			Subject: &ses.Content{
				Charset: aws.String("UTF-8"),
				Data:    aws.String(fmt.Sprintf("Please join %s on Slack!", region)),
			},
		},
		Source: aws.String(from),
	}

	_, err := svc.SendEmail(input)
	if err != nil {
		if aerr, ok := err.(awserr.Error); ok {
			LogPrint(fmt.Sprintf("Error sending email with SES: %q, %q\n", aerr.Code(), aerr.Error()), LogLevelDebug)
			switch aerr.Code() {
			case ses.ErrCodeMailFromDomainNotVerifiedException:
				return EmailTryAgain
			case ses.ErrCodeConfigurationSetDoesNotExistException:
				return EmailTryAgain
			case ses.ErrCodeConfigurationSetSendingPausedException:
				return EmailTryAgain
			case ses.ErrCodeAccountSendingPausedException:
				return EmailTryAgain
			default:
				return EmailFailure
			}
		} else {
			LogPrint(fmt.Sprintf("Error sending email with SES: %q\n", err.Error()), LogLevelDebug)
			return EmailTryAgain
		}
	}
	return EmailSuccess
}
