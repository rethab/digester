<template>
  <v-container>
    <AddSubscription />
    <ListSubscriptions />
  </v-container>
</template>

<script>
import AddSubscription from "@/components/AddSubscription.vue";
import ListSubscriptions from "@/components/ListSubscriptions.vue";
export default {
  components: {
    AddSubscription,
    ListSubscriptions
  }
};
</script>