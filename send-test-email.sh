#!/bin/bash

set -e

source .env.local

curl -s \
-X POST \
-H "Authorization: Bearer $SENDGRID_API_KEY" \
-H 'Content-Type: application/json' \
https://api.sendgrid.com/v3/mail/send \
-d '{
  "from": {
    "email": "digests@digester.app",
    "name": "Digester"
  },
  "template_id": "d-f83856fe31b94f05bff5b81679e56ef0",
  "personalizations": [
    {
      "to": [
        {
          "email": "digesterapp@outlook.com",
          "name": "Reto"
        }
      ],

      "dynamic_template_data": {
        "subject": "Digests from Lady Gaga and NY Times",
        "subscriptions": [
          {"title": "NYT > Top Stories", "updates": [
            {"url": "https://google.nl", "title": "What Does Modern Love Mean in a Pandemic?"}
          ]},
	  {"title": "Lady Gaga", "updates": [
            {"url": "https://google.nl", "title": "I have always wanted to build a company that would support charitable efforts. Starting today, @houslabs will donate $1 from every paid transaction on houslabs.com to @BTWFoundation"}
          ]}
        ]
      }

    }
  ]
  }'
