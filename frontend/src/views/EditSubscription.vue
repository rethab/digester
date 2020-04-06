<template>
  <div>
    <EditSubscription v-if="subscription !== null" v-model="subscription" />
  </div>
</template>

<script>
import EditSubscription from "@/components/subs/EditSubscription.vue";
import Vuex from "@/store/index.js";
export default {
  components: {
    EditSubscription
  },
  data() {
    return {
      subscription: null
    };
  },
  beforeRouteEnter(to, from, next) {
    Vuex.dispatch("loadSubscription", to.params.id).then(subscription => {
      next(vm => vm.setSubscription(subscription));
    });
    // todo error handling
  },
  beforeRouteUpdate(to, from, next) {
    this.subscription = null;
    Vuex.dispatch("loadSubscription", to.params.id).then(subscription => {
      this.setSubscription(subscription);
      next();
    });
    // todo error handling
  },
  methods: {
    setSubscription(subscription) {
      this.subscription = subscription;
    }
  }
};
</script>