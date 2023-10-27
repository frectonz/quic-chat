package main

import (
	"context"
	"crypto/tls"
	"fmt"

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

	// message := "hello"
	// fmt.Printf("Client: Sending '%s'\n", message)
	// _, err = stream.Write([]byte(message))
	// if err != nil {
	// 	return err
	// }

	buf := make([]byte, 512)
	_, err = stream.Read(buf)
	if err != nil {
		return err
	}

	var hello string
	err = msgpack.Unmarshal(buf, &hello)
	if err != nil {
		return err
	}
	fmt.Println("Client: Got", hello)

	return nil
}
