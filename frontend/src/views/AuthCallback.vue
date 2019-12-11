<template>
  <div class="welcome">
    <h1>Welcome, {{ username }}</h1>
  </div>
</template>

<script>
export default {
  computed: {
    username() {
      return this.$store.state.user.username;
    }
  }
};
</script>
