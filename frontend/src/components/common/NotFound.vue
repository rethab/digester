<template>
  <div>
    <v-card class="mx-auto" width="500px" flat>
      <v-card-title>This {{thing}} does not exist..</v-card-title>
      <v-card-subtitle>..but we can help you</v-card-subtitle>
      <v-card-text>
        <ul style="list-style-type: none" class="pl-0">
          <li>
            Back to the
            <router-link :to="link">Overview</router-link>
          </li>
          <li v-if="isAuthenticated">
            Open
            <router-link to="/cockpit">Cockpit</router-link>
          </li>
          <li>
            Visit
            <router-link to="/">digester.app</router-link>
          </li>
        </ul>
      </v-card-text>
    </v-card>
  </div>
</template>

<script>
export default {
  props: {
    thing: {
      type: String,
      required: true
    },
    link: {
      type: String,
      required: true
    }
  },
  computed: {
    isAuthenticated() {
      return this.$store.getters.isAuthenticated;
    }
  }
};
</script>

<style scoped>
li {
  margin-top: 0.5em;
}
</style>