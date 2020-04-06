<template>
  <div>
    <AddSubscription class="py-2" />
    <ListSubscriptions />
  </div>
</template>

<script>
import AddSubscription from "@/components/subs/AddSubscription.vue";
import ListSubscriptions from "@/components/subs/ListSubscriptions.vue";
export default {
  components: {
    AddSubscription,
    ListSubscriptions
  }
};
</script>