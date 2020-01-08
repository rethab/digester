<template>
  <v-container>
    <TimezoneCheck />
    <AddSubscription />
    <ListSubscriptions />
  </v-container>
</template>

<script>
import AddSubscription from "@/components/subs/AddSubscription.vue";
import ListSubscriptions from "@/components/subs/ListSubscriptions.vue";
import TimezoneCheck from "@/components/settings/TimezoneCheck.vue";
export default {
  components: {
    AddSubscription,
    ListSubscriptions,
    TimezoneCheck
  }
};
</script>