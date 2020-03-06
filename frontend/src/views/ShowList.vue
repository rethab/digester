<template>
  <v-container>
    <ShowList v-if="list !== null" v-model="list" />
  </v-container>
</template>

<script>
import ShowList from "@/components/lists/ShowList.vue";
export default {
  components: {
    ShowList
  },
  data() {
    return {
      loading: true,

      list: null
    };
  },
  mounted() {
    this.$store
      .dispatch("loadList", this.$route.params.id)
      .then(list => {
        this.loading = false;
        console.log(list);
        this.list = list;
      })
      .catch(() => {
        this.loading = false;
      });
  }
};
</script>