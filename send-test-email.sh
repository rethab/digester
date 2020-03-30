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
          "email": "rethab@pm.me",
          "name": "Anonymous Panter"
        }
      ],

      "dynamic_template_data": {
        "subject": "Test digest",
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