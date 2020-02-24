<template>
  <div>
    <v-card>
      <v-form @submit.prevent="submit">
        <v-card-title>Create New List</v-card-title>
        <v-card-subtitle>Create and Share Your Own List of Favourite Blogs</v-card-subtitle>
        <v-card-text>
          <v-text-field v-model.trim="name" :error-messages="nameErrors" label="Name"></v-text-field>
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn class="primary" @click="submit">Create</v-btn>
        </v-card-actions>
      </v-form>
    </v-card>
  </div>
</template>

<script>
export default {
  data() {
    return {
      name: "",
      nameErrors: []
    };
  },
  methods: {
    submit() {
      if (this.validate()) {
        this.$store
          .dispatch("createList", { name: this.name })
          .then(list => {
            this.name = "";
            this.$emit("listAdded", list);
          })
          .catch(err => {
            if (err.response.data.error) {
              this.nameErrors.push(err.response.data.error);
            } else {
              this.nameErrors.push("Something went wrong. Please try again.");
            }
          });
      }
    },
    validate() {
      this.nameErrors = [];
      if (this.name.length < 5) {
        this.nameErrors.push("Please make it a bit longer");
      } else if (this.name.length > 30) {
        this.nameErrors.push("The name must be shorter than 30 characters");
      }
      return this.nameErrors.length === 0;
    }
  }
};
</script>
