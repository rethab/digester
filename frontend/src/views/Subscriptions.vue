<template>
  <div>
    <InitializeTimezone v-if="firstLogin" />
    <TimezoneCheck v-else />

    <AddSubscription class="py-2" />
    <ListSubscriptions />
  </div>
</template>

<script>
import AddSubscription from "@/components/subs/AddSubscription.vue";
import ListSubscriptions from "@/components/subs/ListSubscriptions.vue";
import TimezoneCheck from "@/components/settings/TimezoneCheck.vue";
import InitializeTimezone from "@/components/settings/InitializeTimezone.vue";
export default {
  components: {
    AddSubscription,
    ListSubscriptions,
    TimezoneCheck,
    InitializeTimezone
  },
  computed: {
    firstLogin() {
      return this.$route.query.firstLogin;
    }
  }
};
</script>