#!/bin/bash

set -e

source .env.local

curl -s \
-X POST \
--user "$MAILJET_USER:$MAILJET_PASSWORD" \
https://api.mailjet.com/v3.1/send \
-H 'Content-Type: application/json' \
-d '{
  "Messages":[
    {
      "From": {
        "Email": "rethab@rethab.ch",
        "Name": "Ret"
      },
      "To": [
        {
          "Email": "rethab@pm.me",
          "Name": "Ret"
        }
      ],
      "Subject": "Your digest is ready",
      "TemplateID": 1153883,
      "TemplateLanguage": true,
      "Variables": {
        "updates": [{"url": "google.com", "title": "google"}],
	"recipient_name": "rethab"
      }
    }
  ]
}'
