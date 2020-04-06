<template>
  <v-container>
    <v-snackbar v-model="listDeletedSnackbar" :timeout="5000" :top="true">
      List successfully deleted
      <v-btn text @click="listDeletedSnackbar = false" color="error">Close</v-btn>
    </v-snackbar>
    <AddList v-on:listAdded="showEdit" />
    <ListLists />
  </v-container>
</template>

<script>
import AddList from "@/components/lists/AddList.vue";
import ListLists from "@/components/lists/ListLists.vue";
export default {
  components: {
    AddList,
    ListLists
  },
  data() {
    return {
      list: null,
      listDeletedSnackbar: null
    };
  },
  mounted() {
    if (this.$route.query.listDeleted) {
      this.listDeletedSnackbar = true;
    }
  },
  methods: {
    showEdit(list) {
      this.$router.push(`/list/${list.id}/edit`);
    }
  }
};
</script>