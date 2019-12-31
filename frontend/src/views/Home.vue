<template>
  <v-container>
    <h1 class="mt-10 display-3 d-flex justify-center">Welcome to Digester!</h1>
    <AuthLogin v-if="!isAuthenticated" />
    <InitializeTimezone v-if="isAuthenticated && firstLogin" />
    <p
      v-else
      class="d-flex justify-center mt-10"
    >You are logged in. Please use the menu on the left.</p>
  </v-container>
</template>

<script>
import AuthLogin from "@/components/AuthLogin.vue";
import InitializeTimezone from "@/components/InitializeTimezone.vue";
export default {
  data() {
    return {
      firstLogin: this.$route.query.firstLogin
    };
  },
  components: {
    AuthLogin,
    InitializeTimezone
  },
  computed: {
    isAuthenticated() {
      return this.$store.getters.isAuthenticated;
    }
  }
};
</script>
