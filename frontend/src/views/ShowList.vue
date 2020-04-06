<template>
  <v-container>
    <ShowList v-if="list !== null" v-model="list" :dense="false" :checkSubscription="true" />
  </v-container>
</template>

<script>
import ShowList from "@/components/lists/ShowList.vue";
import Vuex from "@/store/index.js";
export default {
  components: {
    ShowList
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