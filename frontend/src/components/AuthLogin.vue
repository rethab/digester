<template>
  <v-container>
    <v-card>
      <v-card-title>Login</v-card-title>
      <v-card-subtitle>
        <span v-if="!!message" class="red--text">{{message}}</span>
        <span v-else>Please login with one of the below methods</span>
      </v-card-subtitle>
      <v-card-text>
        <v-btn dark @click="authenticate('github')">
          Github
          <v-icon small class="pl-1">mdi-github-circle</v-icon>
        </v-btn>
      </v-card-text>
    </v-card>
  </v-container>
</template>

<script>
export default {
  name: "auth-login",
  data() {
    return {
      message: this.$route.query.requireAuth
        ? "This page requires authentication. Please log in:"
        : this.$route.query.sessionExpired
        ? "Your session has expired. Please login again:"
        : null
    };
  },
  methods: {
    authenticate(provider) {
      this.$store
        .dispatch("authenticate", {
          vueAuth: this.$auth,
          provider: provider
        })
        .then(resp => {
          if (resp.data.first_login) {
            this.$router.push({ name: "home", query: { firstLogin: true } });
          } else {
            this.$router.push({ name: "subscriptions" });
          }
        })
        .catch(() => {});
    }
  }
};
</script>