package main

import (
	"context"
	"crypto/tls"
	"fmt"
	"time"

	"github.com/quic-go/quic-go"
	"github.com/vmihailenco/msgpack/v5"
)

const addr = "localhost:5000"

func main() {
	err := clientMain()
	if err != nil {
		panic(err)
	}
}

func clientMain() error {
	tlsConf := &tls.Config{
		InsecureSkipVerify: true,
	}
	conn, err := quic.DialAddr(context.Background(), addr, tlsConf, nil)
	if err != nil {
		return err
	}

	stream, err := conn.AcceptStream(context.Background())
	if err != nil {
		return err
	}

	buf := make([]byte, 512)
	n_bytes, err := stream.Read(buf)
	fmt.Println("client: read", n_bytes, "bytes")
	if err != nil {
		return err
	}

	var hello string
	err = msgpack.Unmarshal(buf, &hello)
	if err != nil {
		return err
	}
	fmt.Println("client: got", hello)

	type PostMessage struct {
		Post []string
	}

	message := PostMessage{[]string{"hello"}}
	buf, err = msgpack.Marshal(&message)
	fmt.Println(buf)
	if err != nil {
		return err
	}
	n_bytes, err = stream.Write(buf)
	fmt.Println("client: sent", n_bytes, "bytes", "len", len(buf))
	if err != nil {
		return err
	}
	fmt.Println("client: sent", message)

	time.Sleep(1 * time.Second)

	return nil
}
