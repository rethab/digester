<template>
  <div>
    <v-btn
      color="black"
      class="white--text"
      :loading="loading"
      :disabled="disabled"
      @click="authenticate('github')"
    >
      Github
      <v-icon small class="pl-1">{{ githubIcon }}</v-icon>
    </v-btn>
    <v-snackbar v-model="toomanyrequests" :timeout="10000" :top="true">
      Our hamsters are turning their wheels as fast they can, but they are currently having some trouble catching up. Please try again in a few minutes.
      <v-btn text @click="snackbar = false" color="error">Close</v-btn>
    </v-snackbar>
  </div>
</template>

<script>
import { mdiGithubCircle } from "@mdi/js";
export default {
  props: {
    disabled: {
      type: Boolean,
      default: false
    }
  },
  data() {
    return {
      loading: false,
      toomanyrequests: false,
      githubIcon: mdiGithubCircle
    };
  },
  methods: {
    authenticate(provider) {
      this.loading = true;
      this.$store
        .dispatch("authenticate", {
          vueAuth: this.$auth,
          provider: provider
        })
        .then(resp => {
          this.loading = false;
          if (resp.data.first_login) {
            this.$router.push({ name: "home", query: { firstLogin: true } });
          } else {
            this.$router.push({ name: "subscriptions" });
          }
        })
        .catch(err => {
          const popupClosed = err.message === "Auth popup window closed";
          if (!popupClosed && !err.response) {
            this.$store.dispatch("setOffline");
          } else if (err.response && err.response.status == 429) {
            this.toomanyrequests = true;
          }
          this.loading = false;
        });
    }
  }
};
</script>