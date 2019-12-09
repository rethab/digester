<template>
  <v-container>
    <h1>Add Blog Page</h1>
    <v-text-field :error-messages="validationErrors" v-model.trim="url" label="Url" />
    <v-btn @click="addBlog" color="primary">Add</v-btn>
  </v-container>
</template>

<script>
import Api from "@/services/api";
export default {
  data() {
    return {
      url: "",
      validationErrors: []
    };
  },
  methods: {
    async addBlog() {
      this.validateUrl();

      if (this.validationErrors.length === 0) {
        try {
          await Api().post("/blogs/add", { url: this.url });
          await this.$store.dispatch("addBlog", this.url);
          this.$router.push({ name: "home" });
        } catch (e) {
          if (e.response.status === 400) {
            this.validationErrors.push(
              "Cannot add this blog: " + e.response.data.error
            );
          } else {
            this.validationErrors.push("Unknown error");
          }
        }
      }
    },
    validateUrl() {
      this.validationErrors = [];
      if (!this.url) {
        this.validationErrors.push("Url is empty");
      }
    }
  }
};
</script>