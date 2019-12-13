<template>
  <v-container>
    <AddDigest />
    <ListDigests />
  </v-container>
</template>

<script>
import AddDigest from "@/components/AddDigest.vue";
import ListDigests from "@/components/ListDigests.vue";
export default {
  components: {
    AddDigest,
    ListDigests
  }
};
</script>