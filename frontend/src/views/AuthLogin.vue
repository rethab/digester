<template>
  <v-container>
    <AuthLogin />
  </v-container>
</template>

<script>
import AuthLogin from "@/components/auth/AuthLogin.vue";
export default {
  components: {
    AuthLogin
  }
};
</script>