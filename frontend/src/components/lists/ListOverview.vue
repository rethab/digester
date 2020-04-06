<template>
  <v-container>
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
      list: null
    };
  },
  methods: {
    showEdit(list) {
      this.$router.push(`/list/${list.id}/edit`);
    }
  }
};
</script>