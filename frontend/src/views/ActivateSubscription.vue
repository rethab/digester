<template>
  <h1>Activating Subscription</h1>
</template>
<script>
import Api from "@/services/api.js";
export default {
  data() {
    return {};
  },
  mounted() {
    let token = this.$route.params.token.trim();
    if (this.validate(token)) {
      this.activate(token)
        .then(() => {
          console.log("Activated :)");
        })
        .catch(err => {
          if (err.response.status == 404) {
            console.log("Not found. Already activated?");
          }
        });
    } else {
      console.log("Invalid token");
    }
  },
  methods: {
    activate(token) {
      return new Promise((resolve, reject) => {
        Api()
          .post(`subscriptions/activate/${token}`)
          .then(() => {
            resolve();
          })
          .catch(err => {
            reject(err);
          });
      });
    },
    validate(token) {
      return /[0-9A-Fa-f]{32}/g.test(token);
    }
  }
};
</script>