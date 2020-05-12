#!/bin/bash

source .env.local

curl -X POST -H "Authorization: Bearer $SLACK_TEST_TOKEN" \
  -H 'Content-type: application/json; charset=utf-8' \
  --data '{
    "channel":"C01467YGJ4Q", 
    "text":"Digests from kubernetes/kubernetes and Lady Gaga",
    "username": "Digester",
    "blocks":[
      { 
        "type": "section",
        "text": {
          "type": "mrkdwn",
          "text": "Hi :wave:, digester has updates for you!"
        }
      },
      { "type": "divider" },
      {
        "type": "section",
        "text": {
          "type": "mrkdwn",
          "text": "*kubernetes/kubernetes*\n\n• <https://github.com/kubernetes/kuberntes|17.0.1>\n• <https://github.com/kubernetes/kuberntes|16.9.1>"
        }
      },
      {
        "type": "section",
        "text": {
          "type": "mrkdwn",
          "text": "*Lady Gaga*\n\n• <https://twitter.com/LadyGaga|Love to you all & hope you love me too!>"
        }
      },
      { "type": "divider" },
      { 
        "type": "section",
        "text": {
          "type": "mrkdwn",
          "text": "_Head over to <https://digester.app|digester.app> to update this subscription or create an additional one_"
        }
      },
    ]
  }' \
  https://slack.com/api/chat.postMessage
