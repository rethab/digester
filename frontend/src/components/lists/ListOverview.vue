<template>
  <v-container>
    <AddList v-on:listAdded="showEdit" />
    <EditList v-if="list" :list="list" :v-on:closeDialog="list = null" />
    <ListLists />
  </v-container>
</template>

<script>
import AddList from "@/components/lists/AddList.vue";
import EditList from "@/components/lists/EditList.vue";
import ListLists from "@/components/lists/ListLists.vue";
export default {
  components: {
    AddList,
    EditList,
    ListLists
  },
  data() {
    return {
      list: null
    };
  },

  methods: {
    showEdit(list) {
      console.log("List: ", list);
      this.list = list;
    }
  }
};
</script>