<template>
  <div class="home">
    <h1>Login</h1>
    <button @click="authenticate('github')">Github Login</button>
  </div>
</template>

<script>
export default {
  name: "auth-login",
  methods: {
    async authenticate(provider) {
      console.log("START authenticate(" + provider + ")");
      try {
        let response = await this.$auth.authenticate(provider);
        let redirect_to = response.data.redirect_to;
        console.log("Redirecting to: " + redirect_to);
        window.location.href = redirect_to;
      } catch (e) {
        console.error("Failed to call authenticate");
        console.error(e);
      }
    }
  }
};
</script>
