<template>
  <v-container>
    <v-card>
      <v-card-title>Add New</v-card-title>
      <v-card-subtitle>
        Into golang? Try
        <span class="font-italic">golang/tools</span>
      </v-card-subtitle>
      <v-card-text>
        <v-row cols="12" align="center">
          <v-col :cols="twoRows ? 4 : 3">
            <v-select :items="types" label="Type"></v-select>
          </v-col>
          <v-col :cols="twoRows ? 8 : 5">
            <v-text-field label="Repository"></v-text-field>
          </v-col>
          <v-col v-if="!twoRows" cols="4">
            <FrequencySelection />
          </v-col>
        </v-row>
        <v-row v-if="twoRows">
          <v-col cols="12">
            <FrequencySelection />
          </v-col>
        </v-row>
      </v-card-text>
      <v-card-actions>
        <v-btn>Clear</v-btn>
        <v-btn class="success">Save</v-btn>
      </v-card-actions>
    </v-card>
  </v-container>
</template>

<script>
import FrequencySelection from "@/components/FrequencySelection.vue";
export default {
  components: {
    FrequencySelection
  },
  data() {
    return {
      types: ["Github", "Youtube", "RSS"],
      twoRows: this.$vuetify.breakpoint.smAndDown
    };
  }
};
</script>
