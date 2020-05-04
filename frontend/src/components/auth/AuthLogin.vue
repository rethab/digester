<template>
  <v-card class="mx-auto" max-width="460px" flat>
    <v-card-text class="pr-0 pl-1">
      <v-row align="center" v-if="showConsentBox">
        <v-checkbox v-model="cookieConsent" hide-details class="mt-0 pt-0" />I agree to the
        <router-link
          target="_blank"
          to="/terms"
          style="color: inherit"
          class="ml-1 font-weight-bold"
        >Terms & Conditions</router-link>
        <span class="mx-1">and</span>
        <router-link
          to="/privacy"
          target="_blank"
          style="color: inherit"
          class="font-weight-bold"
        >Privacy Policy</router-link>
      </v-row>
      <v-row>
        <v-snackbar v-model="toomanyrequests" :timeout="10000" :top="true">
          Our hamsters are turning their wheels as fast they can, but they are currently having some trouble catching up. Please try again in a few minutes.
          <v-btn text @click="snackbar = false" color="error">Close</v-btn>
        </v-snackbar>
      </v-row>
    </v-card-text>
    <v-card-actions>
      <FacebookLoginBtn
        v-on:authenticate="authenticate('facebook')"
        :loading="facebookLoading"
        :disabled="!cookieConsent"
      />
      <v-spacer></v-spacer>
      <GithubLoginBtn
        v-on:authenticate="authenticate('github')"
        :loading="githubLoading"
        :disabled="!cookieConsent"
      />
    </v-card-actions>
    <v-card-text>
      <p>
        Are you neither on Facebook nor on Github? More options are coming soon, but please let us know which one you prefer at
        <a
          href="mailto:info@digester.app"
          style="color: inherit"
        >info@digester.app</a>
      </p>
    </v-card-text>
  </v-card>
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
          this.$router.push({ name: "subscriptions", query: query });
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