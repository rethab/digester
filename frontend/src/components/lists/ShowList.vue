<template>
  <div>
    <EditList v-if="showEdit" :list="value" v-on:closeDialog="showEdit = false" />
    <v-card color="secondary" class="lighten-4">
      <div>
        <v-card-title>{{value.name}}</v-card-title>
        <v-card-text>
          <span v-for="(channel, i) in value.channels" :key="i">
            <span v-if="i > 0">|</span>
            <ChannelIcon :type="channel.type" :small="true" />
            {{channel.name}}
          </span>
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn @click="remove" class="error" text>Delete</v-btn>
          <v-btn @click="edit" class="primary">Edit</v-btn>
        </v-card-actions>
      </div>
    </v-card>
  </div>
</template>

<script>
import ChannelIcon from "@/components/common/ChannelIcon.vue";
import EditList from "@/components/lists/EditList.vue";
export default {
  components: {
    ChannelIcon,
    EditList
  },
  props: {
    value: {
      type: Object,
      required: true
    }
  },
  data() {
    return {
      showEdit: false
    };
  },
  methods: {
    remove() {
      throw "Implement me";
    },
    edit() {
      this.showEdit = true;
    }
  }
};
</script>