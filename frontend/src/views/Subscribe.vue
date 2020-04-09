<template>
  <div>
    <SubscribeForm v-if="channel" :channel="channel" />
    <NotFound v-else thing="Channel" link="/subs" />
  </div>
</template>

<script>
import NotFound from "@/components/common/NotFound.vue";
import SubscribeForm from "@/components/subs/SubscribeForm.vue";
import Api from "@/services/api.js";
export default {
  components: {
    NotFound,
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
      })
      .catch(() => next());
  },
  beforeRouteUpdate(to, from, next) {
    this.channel = null;
    Api()
      .get(`/channels/${to.params.type}/${to.params.id}`)
      .then(resp => {
        this.setChannel(resp.data.channel);
        next();
      })
      .catch(() => next());
  },
  methods: {
    setChannel(channel) {
      this.channel = channel;
    }
  }
};
</script>