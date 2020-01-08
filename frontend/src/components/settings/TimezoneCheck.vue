<template>
  <ErrorCard
    v-if="showWarning"
    title="Missing Timezone"
    message="We're missing your timezone :(. Please click on the button below to update your timezone.
    <br />We need your timezone in order to deliver the digests at the correct time relative to where you live."
    to="/settings"
  />
</template>

<script>
import ErrorCard from "@/components/common/ErrorCard.vue";
export default {
  components: {
    ErrorCard
  },
  data() {
    return {
      showWarning: false
    };
  },
  mounted() {
    this.$store.dispatch("loadTimezone").then(resp => {
      if (!resp.data.timezone) {
        this.showWarning = true;
      }
    });
  }
};
</script>
