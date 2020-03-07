<template>
  <div>
    <EditList v-if="showEdit" :list="value" v-on:closeDialog="showEdit = false" />
    <v-card color="secondary" class="lighten-4">
      <div>
        <v-card-title>
          <router-link
            style="text-decoration: none; color: inherit"
            :to="`/list/${value.id}`"
          >{{value.name}}</router-link>
        </v-card-title>
        <v-card-text>
          <p v-if="value.channels.length == 0">This list contains no channels</p>
          <span v-for="(channel, i) in value.channels" :key="i">
            <span v-if="i > 0">
              <span v-if="dense">|</span>
              <span v-else>
                <br />
              </span>
            </span>
            <ChannelIcon :type="channel.type" :small="true" />
            <a
              style="text-decoration: none; color: inherit"
              target="_blank"
              :href="channel.link"
            >{{channel.name}}</a>
          </span>
        </v-card-text>
        <v-card-actions v-if="userId === value.creatorId">
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
    },
    dense: {
      type: Boolean,
      default: false
    }
  },
  computed: {
    userId() {
      return this.$store.getters.userId;
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