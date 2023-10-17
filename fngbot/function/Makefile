GOOS=linux
GOARCH=amd64
SOURCES=main.go slack.go mailchimp.go ses.go

package.zip: bootstrap
	rm -f package.zip
	zip package.zip bootstrap
bootstrap: main.go slack.go mailchimp.go ses.go
	GOOS=$(GOOS) GOARCH=$(GOARCH) go build -o bootstrap main.go slack.go mailchimp.go ses.go
clean:
	rm -f package.zip bootstrap
