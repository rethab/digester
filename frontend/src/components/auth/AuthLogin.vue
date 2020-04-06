<template>
  <v-container>
    <v-row>
      <v-col cols="12" md="6">
        <p>Privacy Policy</p>
        <p class="overline grey--text">
          <span class="font-weight-bold">Information we collect:</span>&nbsp;
          <span>When using this site, your IP address is recorded for statistics. When logging in, your e-mail address is stored in our database. That e-mail address is used to send digests to, but not shared with any third party whatsoever.</span>
        </p>
        <p class="overline grey--text">
          <span class="font-weight-bold">Cookie Usage:</span>&nbsp;
          <span>When logging in, a cookie is used to identify (remember) you. This cookie is functionally necessary.</span>
        </p>
      </v-col>
      <v-col cols="12" md="6" :order="this.mobile ? 'first' : 'last'">
        <h2 class="title">Available Login Methods</h2>
        <v-checkbox v-model="cookieConsent" v-if="showConsentBox">
          <template v-slot:label>
            <span class="body-2">
              I agree to the
              <strong>Privacy Policy</strong> and the use of
              <strong>Cookies</strong>
            </span>
          </template>
        </v-checkbox>
        <v-row>
          <v-col cols="12" md="6" class="justify-center">
            <FacebookLoginBtn
              v-on:authenticate="authenticate('facebook')"
              :loading="facebookLoading"
              :disabled="!cookieConsent"
            />
          </v-col>
          <v-col cols="12" :md="6" class="justify-center">
            <GithubLoginBtn
              v-on:authenticate="authenticate('github')"
              :loading="githubLoading"
              :disabled="!cookieConsent"
            />
          </v-col>
        </v-row>
        <v-snackbar v-model="toomanyrequests" :timeout="10000" :top="true">
          Our hamsters are turning their wheels as fast they can, but they are currently having some trouble catching up. Please try again in a few minutes.
          <v-btn text @click="snackbar = false" color="error">Close</v-btn>
        </v-snackbar>
      </v-col>
    </v-row>
  </v-container>
</template>

<script>
import GithubLoginBtn from "@/components/auth/GithubLoginBtn.vue";
import FacebookLoginBtn from "@/components/auth/FacebookLoginBtn.vue";

export default {
  name: "auth-login",
  components: {
    GithubLoginBtn,
    FacebookLoginBtn
  },
  data() {
    return {
      mobile: this.$vuetify.breakpoint.smAndDown,

      facebookLoading: false,
      githubLoading: false,

      toomanyrequests: false,

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
  },
  methods: {
    authenticate(provider) {
      const loadingName = `${provider}Loading`;

      this[loadingName] = true;
      this.$store
        .dispatch("authenticate", {
          vueAuth: this.$auth,
          provider: provider
        })
        .then(resp => {
          this[loadingName] = false;

          const query = resp.data.first_login ? { firstLogin: true } : {};
          this.$router.push({ name: "cockpit", query: query });
        })
        .catch(err => {
          const popupClosed = err.message === "Auth popup window closed";
          if (!popupClosed && !err.response) {
            this.$store.dispatch("setOffline");
          } else if (
            err.response &&
            err.response.data.error == "missing_permissions"
          ) {
            this.$router.push({
              name: "auth-login",
              query: { missingPermissions: true }
            });
          } else if (err.response && err.response.status == 429) {
            this.toomanyrequests = true;
          }
          this[loadingName] = false;
        });
    }
  }
};
</script>