<template>
  <v-container>
    <EditList v-if="list !== null" :list="list" />
  </v-container>
</template>

<script>
import EditList from "@/components/lists/EditList.vue";
import Vuex from "@/store/index.js";
export default {
  components: {
    EditList
  },
  data() {
    return {
      list: null
    };
  },
  beforeRouteEnter(to, from, next) {
    Vuex.dispatch("loadList", to.params.id).then(list => {
      next(vm => vm.setList(list));
    });
    // todo error handling
  },
  beforeRouteUpdate(to, from, next) {
    this.list = null;
    Vuex.dispatch("loadList", to.params.id).then(list => {
      this.setList(list);
      next();
    });
    // todo error handling
  },
  methods: {
    setList(list) {
      this.list = list;
    }
  }
};
</script>