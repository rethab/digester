<template>
  <div>
    <EditSubscription v-if="subscription !== null" v-model="subscription" />
    <NotFound v-else thing="Subscription" link="/subs" />
  </div>
</template>

<script>
import NotFound from "@/components/common/NotFound.vue";
import EditSubscription from "@/components/subs/EditSubscription.vue";
import Vuex from "@/store/index.js";
export default {
  components: {
    NotFound,
    EditSubscription
  },
  data() {
    return {
      subscription: null
    };
  },
  beforeRouteEnter(to, from, next) {
    Vuex.dispatch("loadSubscription", to.params.id)
      .then(subscription => {
        next(vm => vm.setSubscription(subscription));
      })
      .catch(() => next());
  },
  beforeRouteUpdate(to, from, next) {
    this.subscription = null;
    Vuex.dispatch("loadSubscription", to.params.id)
      .then(subscription => {
        this.setSubscription(subscription);
        next();
      })
      .catch(() => next());
  },
  methods: {
    setSubscription(subscription) {
      this.subscription = subscription;
    }
  }
};
</script>