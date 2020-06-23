<template>
  <div>
    <h1 class="display-1 d-flex justify-center">Privacy</h1>

    <p class="font-italic text-center py-3">
      This privacy policy explains how Digester uses the personal data we collect from you when you use our website.
      <br />If you have any questions, please contact us at info@digester.app
    </p>

    <v-expansion-panels :multiple="true">
      <v-expansion-panel dark v-for="(section, idx) in sections" :key="idx">
        <v-expansion-panel-header>{{section.title}}</v-expansion-panel-header>
        <v-expansion-panel-content>
          <span v-html="section.content"></span>
        </v-expansion-panel-content>
      </v-expansion-panel>
    </v-expansion-panels>
  </div>
</template>

<script>
export default {
  data() {
    return {
      sections: [
        {
          title: "What data do we collect?",
          content:
            "Digester collects the following data when signing up:" +
            "<ul>" +
            "<li>E-Mail Address</li>" +
            "<li>Name</li>" +
            "<li>Timezone</li>" +
            "</ul>"
        },
        {
          title: "How do we collect your data?",
          content:
            "You directly provide Digest with most of the data we collect." +
            "<br />" +
            "<br />" +
            "We collect data and process data when you:" +
            "<ul>" +
            "<li>Register online for any of our products or services</li>" +
            "</ul>" +
            "<br/>" +
            "Digester may also receive your data indirectly from the following sources:" +
            "<ul>" +
            "<li>Your browser gives us your timezone</li>" +
            "<li>The login provider (eg. when logging in via Facebook, we fetch your e-mail address and name from facebook)</li>" +
            "</ul>"
        },
        {
          title: "How will we use your data?",
          content:
            "Digester collects your data so that we can:<br />" +
            "<ul>" +
            "<li>E-mail you periodic updates (digests)</li>" +
            "<li>Determine at what time to send updates relative to your timezone</li>" +
            "</ul>"
        },
        {
          title: "How do we store your data?",
          content:
            "Digester stores your data at a datacenter of Amazon Webservices in Europe." +
            "<br />" +
            "Digester will keep your data for as long as your account exists. If you delete your account, we will delete your data immediately."
        },
        {
          title: "What are your data protection rights?",
          content:
            "Digester would like to make sure you are fully aware of all of your data protection rights. Every user is entitled to the following:<br />" +
            "<ul>" +
            "<li>The right to access – You have the right to request Digester for copies of your personal data. We may charge you a small fee for this service.</li>" +
            "<li>The right to rectification – You have the right to request that Digester correct any information you believe is inaccurate. You also have the right to request Digester to complete the information you believe is incomplete.</li>" +
            "<li>The right to erasure – You have the right to request that Digester erase your personal data, under certain conditions.</li>" +
            "<li>The right to restrict processing – You have the right to request that Digester restrict the processing of your personal data, under certain conditions.</li>" +
            "<li>The right to object to processing – You have the right to object to Digester’s processing of your personal data, under certain conditions.</li>" +
            "<li>The right to data portability – You have the right to request that Digester transfer the data that we have collected to another organization, or directly to you, under certain conditions.</li>" +
            "</ul>" +
            "<br/>" +
            "If you make a request, we have one month to respond to you. If you would like to exercise any of these rights, please contact us at our email: info@digester.app<br />"
        },
        {
          title: "Third Parties",
          content:
            "In order to provide our services, Digester contracts with a number of companies.<br />" +
            "<ul>" +
            "<li>Netlify Inc (US): <a href='https://www.netlify.com/privacy/' style='color: inherit' target='_blank'>Privacy Policy</a></li>" +
            "<li>Salesforce (US): <a href='https://www.salesforce.com/company/privacy/' style='color: inherit' target='_blank'>Privacy Policy</a></li>" +
            "<li>SolarWinds Worldwide, LLC (US): <a href='https://solarwinds.com/legal/privacy' style='color: inherit' target='_blank'>Privacy Policy</a></li>" +
            "<li>Twilio Inc (US): <a href='https://www.twilio.com/legal/privacy' style='color: inherit' target='_blank'>Privacy Policy</a></li>" +
            "<li>Conva Ventures Inc. (US): <a href='https://usefathom.com/privacy' style='color: inherit' target='_blank'>Privacy Policy</a></li>" +
            "</ul>"
        },
        {
          title: "Cookies",
          content:
            "Cookies are text files placed on your computer to collect standard Internet log information and visitor behavior information. When you visit our websites, we may collect information from you automatically through cookies or similar technology. <br />" +
            "<br />" +
            "For further information, visit <a href='https://allaboutcookies.org' style='color:inherit;' target='_blank'>allaboutcookies.org</a>"
        },
        {
          title: "How do we use cookies?",
          content: "Digester uses Cookies only to keep you signed in."
        },
        {
          title: "What types of cookies do we use?",
          content:
            "There are a number of different types of cookies, however, our website only uses those for functionality in order to keep you signed in."
        },
        {
          title: "How to manage cookies",
          content:
            "You can set your browser not to accept cookies, and the above website tells you how to remove cookies from your browser. However, if you do so, you won't be able to login to Digester."
        },
        {
          title: "Privacy policies of other websites",
          content:
            "The Digester website contains links to other websites. Our privacy policy applies only to our website, so if you click on a link to another website, you should read their privacy policy."
        },
        {
          title: "Changes to our privacy policy",
          content:
            "Digester keeps its privacy policy under regular review and places any updates on this web page. This privacy policy was last updated on 23 June 2020."
        },
        {
          title: "How to contact us",
          content:
            "If you have any questions about Digester's privacy policy, the data we hold on you, or you would like to exercise one of your data protection rights, please do not hesitate to contact us." +
            "<br />" +
            "<br />" +
            "Email us at: info@digester.app" +
            "<br />" +
            "<br />" +
            "Or write to us at: Digester, c/o Reto Habluetzel, Lizzy Ansinghstraat 163, 1072 RG Amsterdam"
        }
      ]
    };
  }
};
</script>
