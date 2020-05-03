<template>
  <div>
    <h1 class="display-1 d-flex justify-center">Terms & Conditions</h1>

    <p
      class="font-italic text-center pt-3"
    >These terms and conditions outline the rules and regulations for the use of Digester's Website.</p>
    <p class="font-italic text-center pb-3">
      By accessing this website we assume you accept these terms and conditions in full. Do not continue to use Digester's website
      if you do not accept all of the terms and conditions stated on this page.
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
          title: "Limitations of Liability",
          content:
            "Digester is not liable for any actions you may take based on content on our website." +
            "<br />" +
            "We have no liability for any loss or damage of any sort caused by malfunctioning of Digester. " +
            "In particular, in case a digest (weekly or daily update) is (a) not sent out, (b) sent out with wrong contents, (c) " +
            "sent out with incomplete content, or (d) sent out at the wrong time."
        },
        {
          title: "Your Content in our Service",
          content:
            "Some of our services allow you to create content. When you create content on our website, you give Digester a " +
            "worldwide license to use, host, store, reproduce, modify, create derivative work (eg. translations), communicte, publish and distribute that content." +
            "<br />" +
            "The rights you grant are used to operate our services and the license does not expire when terminating the account."
        },
        {
          title: "Removal of Links from our Website",
          content:
            "If you find any link on our Web site or any linked web site objectionable for any reason, you may contact" +
            "us about this. We will consider requests to remove links but will have no obligation to do so or to respond" +
            "directly to you." +
            "<br />" +
            "Whilst we endeavour to ensure that the information on this website is correct, we do not warrant its completeness" +
            "or accuracy; nor do we commit to ensuring that the website remains available or that the material on the" +
            "website is kept up to date."
        },
        {
          title: "Suspension and Termination of Service",
          content:
            "Digester may limit, suspend, or stop proving the services to you without notice if you fail to " +
            "comply (or are suspected to) with these terms, or if you use" +
            "the service in a way that causes legal liability or is disruptive to Digester or any other user."
        },
        {
          title: "Cookies",
          content:
            "We employ the use of cookies. By using Digester's website you consent to the use of cookies in accordance with Digester's privacy policy."
        },
        {
          title: "Content Liability",
          content:
            "We shall have no responsibility or liability for any content appearing on your Web site. You agree to indemnify" +
            "and defend us against all claims arising out of or based upon your Website. No link(s) may appear on any" +
            "page on your Web site or within any context containing content or materials that may be interpreted as" +
            "libelous, obscene or criminal, or which infringes, otherwise violates, or advocates the infringement or" +
            "other violation of, any third party rights."
        },
        {
          title: "Disclaimer",
          content:
            "To the maximum extent permitted by applicable law, we exclude all representations, warranties and conditions relating to our website and the use of this website (including, without limitation, any warranties implied by law in respect of satisfactory quality, fitness for purpose and/or the use of reasonable care and skill). Nothing in this disclaimer will:" +
            "<ul>" +
            "<li>limit or exclude our or your liability for death or personal injury resulting from negligence;</li>" +
            "<li>limit or exclude our or your liability for fraud or fraudulent misrepresentation;</li>" +
            "<li>limit any of our or your liabilities in any way that is not permitted under applicable law; or</li>" +
            "<li>exclude any of our or your liabilities that may not be excluded under applicable law.</li>" +
            "<ul>"
        },
        {
          title: "Governing Law",
          content: "These terms will be governed by Dutch law."
        },
        {
          title: "Entire Agreement",
          content:
            "These terms constitute the entire agreement between you and Digester with respect to the subject matter of these Terms, and " +
            "supersede and replace any other prio or contemporaneous agreements, or terms and conditions applicable to the subject matter of these Terms." +
            "These terms create no third party beneficiary rights."
        }
      ]
    };
  }
};
</script>
