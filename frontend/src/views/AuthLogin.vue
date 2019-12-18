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
      let response = await this.$auth.authenticate(provider);
      await this.$store.dispatch("authenticate", {
        username: response.data.username
      });
      this.$router.push({ name: "subscriptions" });
    }
  }
};
</script>
