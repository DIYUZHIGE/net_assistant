<template>
  <div class="container">
    <el-tabs class="all-tabs">
      <el-tab-pane label="TCP Client" name="TCP Client">
        <el-form :model="tcp_client_form" label-width="auto" style="max-width: 100%">
          <el-form-item label="IP地址">
            <el-input v-model="tcp_client_form.ip" placeholder="请输入IP地址" />
          </el-form-item>
          <el-form-item label="端口号">
            <el-input v-model="tcp_client_form.port" placeholder="请输入端口号" />
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="tcp_client_control">{{ tcp_client_form.status }}</el-button>
          </el-form-item>
          <el-form-item label="数据历史">
            <el-input type="textarea" v-model="tcp_client_form.data_history" placeholder="数据历史" readonly />
          </el-form-item>
          <el-form-item label="消息">
            <el-input v-model="tcp_client_form.message" placeholder="请输入消息" />
          </el-form-item>
          <el-form-item>
            <el-button type="success" @click="tcp_client_send">发送</el-button>
          </el-form-item>
        </el-form>
      </el-tab-pane>
      <el-tab-pane label="TCP Server" name="TCP Server">
        <el-form :model="tcp_server_form" label-width="auto" style="max-width: 100%">
          <el-form-item label="IP地址">
            <el-input v-model="tcp_server_form.ip" placeholder="请输入IP地址" />
          </el-form-item>
          <el-form-item label="端口号">
            <el-input v-model="tcp_server_form.port" placeholder="请输入端口号" />
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="tcp_server_control">{{ tcp_server_form.status }}</el-button>
          </el-form-item>
          <el-form-item label="数据历史">
            <el-input type="textarea" v-model="tcp_server_form.data_history" placeholder="数据历史" readonly />
          </el-form-item>
          <el-form-item label="消息">
            <el-input v-model="tcp_server_form.message" placeholder="请输入消息" />
          </el-form-item>
          <el-form-item>
            <el-button type="success" @click="tcp_server_send">发送</el-button>
          </el-form-item>
        </el-form>
      </el-tab-pane>
      <el-tab-pane label="UDP" name="UDP">UDP</el-tab-pane>
      <el-tab-pane label="COM" name="COM">COM</el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue'
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

type update_tcp_client_status = {
  status: string
};
type update_tcp_server_status = {
  status: string
};

type update_tcp_server_history = {
  history: string
}

type update_tcp_client_history = {
  history: string
}

listen<update_tcp_client_status>('update_tcp_client_status', async (event) => {
  tcp_client_form.status = event.payload.status;
});
listen<update_tcp_client_history>('update_tcp_client_history', async (event) => {
  tcp_client_form.data_history = event.payload.history;
});
listen<update_tcp_server_status>('update_tcp_server_status', async (event) => {
  tcp_server_form.status = event.payload.status;
});

listen<update_tcp_server_history>('update_tcp_server_history', async (event) => {
  tcp_server_form.data_history = event.payload.history;
});




// do not use same name with ref
const tcp_client_form = reactive({
  ip: '127.0.0.1',
  port: '8080',
  message: '',
  data_history: '',
  status: 'disconnected'
})

const tcp_server_form = reactive({
  ip: '127.0.0.1',
  port: '8080',
  status: 'destroyed',
  message: '',
  data_history: ''
})

const tcp_client_control = async () => {
  if (tcp_client_form.status === 'disconnected') {
    invoke('update_tcp_client_form', {
      ip: tcp_client_form.ip,
      port: tcp_client_form.port
    })
    invoke('tcp_client_connect')
  } else if (tcp_client_form.status === 'connected') {
    invoke('tcp_client_disconnect')
  }
}

const tcp_client_send = async () => {
  invoke('tcp_client_send', {
    message: tcp_client_form.message
  })
}

const tcp_server_control = async () => {
  if (tcp_server_form.status === 'destroyed') {
    await invoke('update_tcp_server_form', {
      ip: tcp_server_form.ip,
      port: tcp_server_form.port
    })
    invoke('tcp_server_establish')
  } else if (tcp_server_form.status === 'established') {
    invoke('tcp_server_destroy')
  }
}

const tcp_server_send = async () => {
  invoke('tcp_server_send', {
    message: tcp_server_form.message
  })
}
</script>

<style></style>