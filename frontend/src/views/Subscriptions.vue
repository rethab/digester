<template>
  <v-container>
    <TimezoneCheck />
    <AddSubscription />
    <ListSubscriptions />
  </v-container>
</template>

<script>
import AddSubscription from "@/components/AddSubscription.vue";
import ListSubscriptions from "@/components/ListSubscriptions.vue";
import TimezoneCheck from "@/components/TimezoneCheck.vue";
export default {
  components: {
    AddSubscription,
    ListSubscriptions,
    TimezoneCheck
  }
};
</script>