<template>
  <v-container>
    <div class="text-center">
      <h1 class="display-1">Updates</h1>
      <p
        class="subtitle-1 font-italic grey--text"
      >Since updates are fetched periodically, it might take some time until they appear here.</p>
    </div>
    <ListUpdates />
  </v-container>
</template>

<script>
import ListUpdates from "@/components/updates/ListUpdates.vue";
export default {
  components: {
    ListUpdates
  }
};
</script>