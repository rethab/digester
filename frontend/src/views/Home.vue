<template>
  <v-container>
    <h1 class="mt-10 display-3 d-flex justify-center">Welcome to Digester!</h1>
    <AuthLogin />
  </v-container>
</template>

<script>
import AuthLogin from "@/components/AuthLogin.vue";
export default {
  components: {
    AuthLogin
  }
};
</script>
