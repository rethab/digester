<template>
  <v-card>
    <v-card-title>Create New List</v-card-title>
    <v-card-subtitle>Create and Share Your Own List of Favourite Blogs</v-card-subtitle>
    <v-card-text>
      <v-form @submit.prevent="submit">
        <v-text-field v-model.trim="name" :error-messages="nameErrors" label="Name"></v-text-field>
        <v-text-field v-model.trim="name" :error-messages="nameErrors" label="Add Blog"></v-text-field>
      </v-form>
      <ul>
        <li>The Scala Blog</li>
        <li>Seven Myths about Dotty</li>
        <li>SBT Daily</li>
      </ul>
    </v-card-text>
    <v-card-actions>
      <v-spacer></v-spacer>
      <v-btn>Save</v-btn>
    </v-card-actions>
  </v-card>
</template>

<script>
export default {
  components: {},
  data() {
    return {
      successSnackbar: false,
      errorSnackbar: false,
      errorMessage: null,

      name: null,
      nameErrors: []
    };
  }
};
</script>
