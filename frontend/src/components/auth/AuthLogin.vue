<template>
  <v-container>
    <v-row>
      <v-col cols="12" md="6">
        <p>Privacy Policy</p>
        <p class="overline grey--text">
          <span class="font-weight-bold">How Information is used:</span>&nbsp;
          <span>From the login provider, we are going to retrieve your e-mail address and store it in our database. We use that e-mail address to send digests to. This information is not shared with any third party whatsoever.</span>
        </p>
        <p class="overline grey--text">
          <span class="font-weight-bold">Cookie Usage:</span>&nbsp;
          <span>When logging in, cookies are used to identify your session. These cookies are functionally necessary.</span>
        </p>
      </v-col>
      <v-col cols="12" md="6" :order="this.mobile ? 'first' : 'last'">
        <h2 class="title">Available Login Methods*</h2>
        <p class="font-italic caption">* More coming soon</p>
        <v-checkbox v-model="cookieConsent" v-if="showConsentBox">
          <template v-slot:label>
            <span class="body-2">
              I agree to the
              <strong>Privacy Policy</strong> and the use of
              <strong>Cookies</strong>
            </span>
          </template>
        </v-checkbox>
        <GithubLoginBtn :disabled="!cookieConsent" />
      </v-col>
    </v-row>
  </v-container>
</template>

<script>
import GithubLoginBtn from "@/components/auth/GithubLoginBtn.vue";

export default {
  name: "auth-login",
  components: {
    GithubLoginBtn
  },
  data() {
    return {
      mobile: this.$vuetify.breakpoint.smAndDown,

      showConsentBox: true,
      cookieConsent: false
    };
  },
  mounted() {
    // if the user gave consent before (ie. the value from
    // localStorage is true at the time of loading), we don't
    // even show the checkbox anymore.
    if (localStorage.cookieConsent) {
      const cookieConsent = localStorage.cookieConsent == "true";
      this.showConsentBox = cookieConsent !== true;
      this.cookieConsent = cookieConsent === true;
    }
  },
  watch: {
    cookieConsent(newValue) {
      localStorage.setItem("cookieConsent", newValue);
    }
  }
};
</script>