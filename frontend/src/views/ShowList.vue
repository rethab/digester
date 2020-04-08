<template>
  <div>
    <ShowList v-if="list !== null" v-model="list" :dense="false" :checkSubscription="true" />
    <NotFound v-else thing="List" link="/lists" />
  </div>
</template>

<script>
import NotFound from "@/components/common/NotFound.vue";
import ShowList from "@/components/lists/ShowList.vue";
import Vuex from "@/store/index.js";
export default {
  components: {
    NotFound,
    ShowList
  },
  data() {
    return {
      list: null
    };
  },
  beforeRouteEnter(to, from, next) {
    Vuex.dispatch("loadList", to.params.id)
      .then(list => {
        next(vm => vm.setList(list));
      })
      .catch(() => next());
  },
  beforeRouteUpdate(to, from, next) {
    this.list = null;
    Vuex.dispatch("loadList", to.params.id)
      .then(list => {
        this.setList(list);
        next();
      })
      .catch(() => next());
  },
  methods: {
    setList(list) {
      this.list = list;
    }
  }
};
</script>