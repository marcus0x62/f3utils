package main

import (
	"fmt"
	"net/url"
	"strings"
)

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

func main() {
	string := "token=RDhBgDFgfFgMLxm9apiOLqKo\u0026team_id=T4W26HAU9\u0026team_domain=marcus0x62\u0026channel_id=D4WP932G4\u0026channel_name=directmessage\u0026user_id=U4WL42U91\u0026user_name=marcusb\u0026command=%2Ftestinvoke\u0026text=\u0026api_app_id=A05HS3CM6R5\u0026is_enterprise_install=false\u0026response_url=https%3A%2F%2Fhooks.slack.com%2Fcommands%2FT4W26HAU9%2F5621866837988%2FteGKv01xUglHHiWD0Yw5sqIw\u0026trigger_id=5643128783472.166074588961.e4ad2c450e9a3032276c88ce262e6d10"

	inputmap := parse_args(string)
	for key := range inputmap {
		fmt.Printf("%s: %s\n", key, inputmap[key])
	}
}
