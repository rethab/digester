<template>
  <div>
    <div class="text-center">
      <h1 class="title mt-2" v-if="lists.length > 0">My Lists</h1>
    </div>
    <v-container fluid>
      <v-overlay absolute opacity="0.1" :value="loading">
        <v-progress-circular indeterminate color="primary" size="64"></v-progress-circular>
      </v-overlay>
      <v-row dense>
        <v-col v-for="(list, idx) in lists" :key="idx" cols="12">
          <ShowList :value="list" />
        </v-col>
      </v-row>
    </v-container>
  </div>
</template>

<script>
import ShowList from "@/components/lists/ShowList.vue";
import { mapGetters } from "vuex";
export default {
  data() {
    return {
      loading: true,
      error: false
    };
  },
  components: {
    ShowList
  },
  computed: {
    ...mapGetters(["lists"])
  },
  mounted() {
    this.$store
      .dispatch("loadLists")
      .then(() => {
        this.loading = false;
      })
      .catch(() => {
        this.loading = false;
        this.error = true;
      });
  }
};
</script>
