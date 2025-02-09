<template>
  <div class="container">
    <el-tabs class="all-tabs">
      <el-tab-pane label="TCP Client" name="TCP Client">
        <el-form :model="tcp_client_form" label-width="auto" style="max-width: 100%">
          <el-form-item label="IP地址" label-position="top">
            <el-input v-model="tcp_client_form.ip" placeholder="请输入IP地址" />
          </el-form-item>
          <el-form-item label="端口号" label-position="top">
            <el-input v-model="tcp_client_form.port" placeholder="请输入端口号" />
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="tcp_client_control" style="width: 100%">{{ tcp_client_form.status ==
              'disconnected' ? '连接' : '断开' }}</el-button>
          </el-form-item>
          <el-form-item label="数据历史" label-position="top">
            <el-input type="textarea" v-model="tcp_client_form.data_history" placeholder="数据历史" :rows="4"
              style="height: 100%" readonly />
          </el-form-item>
          <el-form-item label="消息" label-position="top">
            <el-input v-model="tcp_client_form.message" placeholder="请输入消息" />
          </el-form-item>
          <el-form-item>
            <el-button type="success" @click="tcp_client_send" style="width: 100%">发送</el-button>
          </el-form-item>
        </el-form>
      </el-tab-pane>
      <el-tab-pane label="TCP Server" name="TCP Server">
        <el-form :model="tcp_server_form" label-width="auto" style="max-width: 100%">
          <el-form-item label="IP地址" label-position="top">
            <el-input v-model="tcp_server_form.ip" placeholder="请输入IP地址" />
          </el-form-item>
          <el-form-item label="端口号" label-position="top">
            <el-input v-model="tcp_server_form.port" placeholder="请输入端口号" />
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="tcp_server_control" style="width: 100%">{{ tcp_server_form.status ==
              'destroyed' ? '建立' : '断开' }}</el-button>
          </el-form-item>
          <el-form-item label="数据历史" label-position="top">
            <el-input type="textarea" v-model="tcp_server_form.data_history" placeholder="数据历史" :rows="4"
              style="height: 100%" readonly />
          </el-form-item>
          <el-form-item label="消息" label-position="top">
            <el-input v-model="tcp_server_form.message" placeholder="请输入消息" />
          </el-form-item>
          <el-form-item>
            <el-button type="success" @click="tcp_server_send" style="width: 100%">发送</el-button>
          </el-form-item>
        </el-form>
      </el-tab-pane>
      <el-tab-pane label="UDP" name="UDP">
        <el-form :model="udp_form" label-width="auto" style="max-width: 100%">
          <el-form-item label="IP地址" label-position="top">
            <el-input v-model="udp_form.ip" placeholder="请输入IP地址" />
          </el-form-item>
          <el-form-item label="端口号" label-position="top">
            <el-input v-model="udp_form.port" placeholder="请输入端口号" />
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="udp_control" style="width: 100%">{{ udp_form.status == 'unbinded' ? '绑定' :
              '解绑' }}</el-button>
          </el-form-item>
          <el-form-item label="目标IP地址" label-position="top">
            <el-input v-model="udp_form.to_ip" placeholder="请输入目标IP地址" />
          </el-form-item>
          <el-form-item label="目标端口号" label-position="top">
            <el-input v-model="udp_form.to_port" placeholder="请输入目标端口号" />
          </el-form-item>
          <el-form-item label="数据历史" label-position="top">
            <el-input type="textarea" v-model="udp_form.data_history" placeholder="数据历史" :rows="4" style="height: 100%"
              readonly />
          </el-form-item>
          <el-form-item label="消息" label-position="top">
            <el-input v-model="udp_form.message" placeholder="请输入消息" />
          </el-form-item>
          <el-form-item>
            <el-button type="success" @click="udp_send" style="width: 100%">发送</el-button>
          </el-form-item>
        </el-form>
      </el-tab-pane>
      <el-tab-pane label="Serial Port" name="Serial Port">
        <el-form :model="serial_port_form" label-width="auto" style="max-width: 100%">

          <el-form-item label="串口" label-position="top">
            <el-select v-model="serial_port_form.port" placeholder="请选择串口" @visible-change="serial_port_get_ports">
              <el-option v-for="port in serial_ports" :key="port" :label="port" :value="port" />
            </el-select>
          </el-form-item>
          <el-form-item label="波特率" label-position="top">
            <el-select v-model="serial_port_form.baudrate" placeholder="请选择波特率">
              <el-option label="1200" value="1200" />
              <el-option label="2400" value="2400" />
              <el-option label="4800" value="4800" />
              <el-option label="9600" value="9600" />
              <el-option label="19200" value="19200" />
              <el-option label="38400" value="38400" />
              <el-option label="57600" value="57600" />
              <el-option label="115200" value="115200" />
            </el-select>
          </el-form-item>
          <el-form-item label="数据位" label-position="top">
            <el-input-number v-model="serial_port_form.data_bits" />
          </el-form-item>
          <el-form-item label="停止位" label-position="top">
            <el-input-number v-model="serial_port_form.stop_bits" />
          </el-form-item>
          <el-form-item label="校验位" label-position="top">
            <el-select v-model="serial_port_form.parity" placeholder="请选择校验位">
              <el-option label="无校验" value="none" />
              <el-option label="奇校验" value="odd" />
              <el-option label="偶校验" value="even" />
            </el-select>
          </el-form-item>
          <el-form-item label="流控" label-position="top">
            <el-select v-model="serial_port_form.flow_control" placeholder="请选择流控">
              <el-option label="无流控" value="none" />
              <el-option label="软件流控" value="software" />
              <el-option label="硬件流控" value="hardware" />
            </el-select>
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="serial_port_control" style="width: 100%">
              {{ serial_port_form.status == 'closed' ? '打开' : '关闭' }}
            </el-button>
          </el-form-item>

          <el-form-item label="数据历史" label-position="top">
            <el-input type="textarea" v-model="serial_port_form.history" placeholder="数据历史" :rows="4"
              style="height: 100%" readonly />
          </el-form-item>
          <el-form-item label="消息" label-position="top">
            <el-input v-model="serial_port_form.message" placeholder="请输入消息" />
          </el-form-item>
          <el-form-item>
            <el-button type="success" @click="serial_port_send" style="width: 100%">发送</el-button>
          </el-form-item>
        </el-form>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
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

type update_udp_status = {
  status: string
}

type update_udp_history = {
  history: string
}

type update_serial_ports = {
  names: string[]
}

type update_serial_port_status = {
  status: string
}

type update_serial_port_history = {
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

listen<update_udp_status>('update_udp_status', async (event) => {
  udp_form.status = event.payload.status;
});

listen<update_udp_history>('update_udp_history', async (event) => {
  udp_form.data_history = event.payload.history;
});

listen<update_serial_ports>('update_serial_port_names', async (event) => {
  serial_ports.value = event.payload.names;
});

listen<update_serial_port_status>('update_serial_port_status', async (event) => {
  serial_port_form.status = event.payload.status;
});

listen<update_serial_port_history>('update_serial_port_history', async (event) => {
  serial_port_form.history = event.payload.history;
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

const udp_form = reactive({
  ip: '127.0.0.1',
  port: '8080',
  to_ip: '127.0.0.1',
  to_port: '8081',
  status: 'unbinded',
  message: '',
  data_history: ''
})



const udp_control = async () => {
  if (udp_form.status === 'unbinded') {
    await invoke('update_udp_form', {
      ip: udp_form.ip,
      port: udp_form.port,
      toIp: udp_form.to_ip,
      toPort: udp_form.to_port
    })
    invoke('udp_bind')
  } else if (udp_form.status === 'binded') {
    invoke('udp_unbind')
  }
}

const udp_send = async () => {
  invoke('udp_send', {
    message: udp_form.message
  })
}


//serial port


const serial_port_form = reactive({
  port: '',
  baudrate: '115200',
  data_bits: 8,
  stop_bits: 1,
  parity: 'none',
  flow_control: 'none',
  status: 'closed',
  history: '',
  message: ''
})

let serial_ports = ref(['COM1', 'COM2'])

const serial_port_control = async () => {
  if (serial_port_form.status === 'closed') {
    await invoke('update_serial_port_parameters', {
      port: serial_port_form.port,
      baudRate: serial_port_form.baudrate,
      dataBits: serial_port_form.data_bits,
      stopBits: serial_port_form.stop_bits,
      parity: serial_port_form.parity,
      flowControl: serial_port_form.flow_control
    })
    invoke('serial_port_open')
  } else if (serial_port_form.status === 'opened') {
    invoke('serial_port_close')
  }
}

const serial_port_send = async () => {
  invoke('serial_port_write', {
    message: serial_port_form.message
  })
}
invoke('serial_ports_refresh')
const serial_port_get_ports = async () => {
  await invoke('serial_ports_refresh')
}
</script>

<style scoped>
</style>
