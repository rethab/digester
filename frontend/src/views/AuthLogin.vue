<template>
  <v-container>
    <v-row justify="center">
      <v-col class="text-center">
        <h1 class="display-2 font-weight-medium">Login</h1>
      </v-col>
    </v-row>
    <ErrorCard v-if="!!message" title="Login Required" :message="message" />
    <AuthLogin />
  </v-container>
</template>

<script>
import AuthLogin from "@/components/auth/AuthLogin.vue";
import ErrorCard from "@/components/common/ErrorCard.vue";
export default {
  components: {
    AuthLogin,
    ErrorCard
  },
  data() {
    return {
      message: this.$route.query.requireAuth
        ? "This page requires authentication. Please log in."
        : this.$route.query.sessionExpired
        ? "Your session has expired. Please login again."
        : null
    };
  },

  mounted() {
    const isAuthenticated = this.$store.getters.isAuthenticated;
    if (isAuthenticated) {
      this.$router.push({ name: "home" });
    }
  }
};
</script>