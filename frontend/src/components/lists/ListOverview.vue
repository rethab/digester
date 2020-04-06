<template>
  <div>
    <AddList v-on:listAdded="showEdit" />
    <ListLists />
  </div>
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