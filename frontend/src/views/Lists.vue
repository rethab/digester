<template>
  <v-container>
    <AddList />
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
  }
};
</script>