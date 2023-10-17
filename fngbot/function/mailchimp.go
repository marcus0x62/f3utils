/*
 * MailChimp API calls
 */

package main

import (
	"encoding/json"
	"fmt"
	"io"
	. "marcusb.org/golang/debug"
	"net/http"
	"strings"
)

type MailChimpResponseError struct {
	Email_Address string
	Error         string
	Error_Code    string
}

type MailChimpResponse struct {
	Errors []MailChimpResponseError
}

const (
	MailChimpSuccess = iota
	MailChimpExists  = iota
	MailChimpFailure = iota
)

var MailChimpAddPayload string = `
{"members": [{"email_address": "%s",
              "status": "subscribed",
              "email_type": "html",
              "merge_fields": {"FULLNAME": "%s",
                               "PHONE": "%s",
                               "F3NAME": "%s"
                              }
            }],
            "sync_tags": false,
            "update_existing": false
}`

func mailchimp_add(f3_name string, hospital_name string, email string, cell string) int {
	var mcresponse MailChimpResponse

	httpclient := &http.Client{}

	/* Add FNG to Mail Chimp */
	req, _ := http.NewRequest("POST",
		fmt.Sprintf("https://%s/3.0/lists/%s?skip_merge_validation-true", ENV_MAILCHIMP_API_ENDPOINT, ENV_MAILCHIMP_LIST_ID),
		strings.NewReader(fmt.Sprintf(MailChimpAddPayload,
			email, hospital_name, cell, f3_name)))
	req.SetBasicAuth("anystring", ENV_MAILCHIMP_API_KEY)

	resp, err := httpclient.Do(req)
	if err != nil {
		LogPrint(fmt.Sprintf("Error sending POST request to MailChimp: %q", err), LogLevelDebug)
		return MailChimpFailure
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		LogPrint(fmt.Sprintf("Error reading Mailchimp response body: %q %q\n", body, err), LogLevelDebug)
		return MailChimpFailure
	}

	if resp.StatusCode != http.StatusOK {
		LogPrint(fmt.Sprintf("HTTP error code from Mailchimp API: %q\n", resp.Status), LogLevelDebug)
		return MailChimpFailure
	}

	json.Unmarshal([]byte(body), &mcresponse)
	for _, mcerr := range mcresponse.Errors {
		LogPrint(fmt.Sprintf("Mailchimp returned an error: %s\n", mcerr.Error), LogLevelDebug)

		if mcerr.Error_Code == "ERROR_CONTACT_EXISTS" {
			return MailChimpExists
		} else {
			return MailChimpFailure
		}
	}

	return MailChimpSuccess
}
