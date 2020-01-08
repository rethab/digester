<template>
  <v-container>
    <v-card>
      <v-card-title>Login</v-card-title>
      <v-card-subtitle>
        <span v-if="!!message" class="red--text">{{message}}</span>
        <span v-else>Please login with one of the below methods</span>
      </v-card-subtitle>
      <v-card-text>
        <GithubLoginBtn />
      </v-card-text>
    </v-card>
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
      message: this.$route.query.requireAuth
        ? "This page requires authentication. Please log in:"
        : this.$route.query.sessionExpired
        ? "Your session has expired. Please login again:"
        : null
    };
  }
};
</script>