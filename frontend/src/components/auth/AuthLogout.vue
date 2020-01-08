<template><div class="logout"/></template>

<script>
export default {
  created() {
    this.$store.dispatch("unauthenticate").then(() => {
      this.$router.push({ name: "home" });
    });
  }
};
</script>
