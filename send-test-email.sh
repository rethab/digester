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
        "Email": "digests@digester.app",
        "Name": "Digester"
      },
      "To": [
        {
          "Email": "rethab@pm.me",
          "Name": "Anonymous Panter"
        }
      ],
      "Subject": "Your digest is ready",
      "TemplateErrorReporting": {
        "Email": "rethab@rethab.ch",
        "Name": "Reto"
      },
      "TemplateErrorDeliver": false,
      "TemplateID": 1153883,
      "TemplateLanguage": true,
      "Variables": {
        "update_subscriptions_url": "https://google.com",
        "add_subscription_url": "https://google.com",
        "subscriptions": [
	        {"title": "kubernetes/kubernetes", "updates": [
            {"url": "https://google.nl", "title": "v1.18.0-alpha.1"},
            {"url": "https://google.nl", "title": "v1.14.10"}
          ]},
          {"title": "golang/tools", "updates": [
            {"url": "https://google.nl", "title": "gopls/v0.2.2"},
            {"url": "https://google.nl", "title": "gopls/v0.1.6"}
          ]}
        ]
      }
    }
  ]
}'
