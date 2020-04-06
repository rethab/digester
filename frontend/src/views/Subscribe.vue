<template>
  <div>
    <SubscribeForm v-if="channel" :channel="channel" />
  </div>
</template>

<script>
import SubscribeForm from "@/components/subs/SubscribeForm.vue";
import Api from "@/services/api.js";
export default {
  components: {
    SubscribeForm
  },
  data() {
    return {
      channel: null
    };
  },
  beforeRouteEnter(to, from, next) {
    Api()
      .get(`/channels/${to.params.type}/${to.params.id}`)
      .then(resp => {
        next(vm => vm.setChannel(resp.data));
      });
    // todo error handling
  },
  beforeRouteUpdate(to, from, next) {
    this.channel = null;
    Api()
      .get(`/channels/${to.params.type}/${to.params.id}`)
      .then(resp => {
        this.setChannel(resp.data.channel);
        next();
      });
    // todo error handling
  },
  methods: {
    setChannel(channel) {
      this.channel = channel;
    }
  }
};
</script>