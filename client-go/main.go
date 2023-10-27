package main

import (
	"context"
	"crypto/tls"
	"fmt"
	"io"

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
	var buf []byte

	// 1. read hello message
	buf, err = read(stream)

	var hello string
	err = msgpack.Unmarshal(buf, &hello)
	if err != nil {
		return err
	}
	fmt.Println("client: got", hello)
	assert(hello == "Hello")

	// 2. write post message
	type PostMessage struct {
		Post []string
	}

	message := PostMessage{[]string{"hello"}}
	buf, err = msgpack.Marshal(&message)
	if err != nil {
		return err
	}

	err = write(stream, buf)
	if err != nil {
		return err
	}

	// 3. read ok message
	buf, err = read(stream)
	if err != nil {
		return err
	}

	var ok string
	err = msgpack.Unmarshal(buf, &ok)
	if err != nil {
		return err
	}
	fmt.Println("client: got", ok)
	assert(ok == "OK")

	return nil
}

func assert(check bool) {
	if !check {
		panic("that wasn't supposed to happen")
	}
}

func read(stream quic.Stream) ([]byte, error) {
	buf := make([]byte, 512)
	n_bytes, err := stream.Read(buf)
	fmt.Println("client: read", n_bytes, "bytes")

	if err == io.EOF {
		return buf, nil
	}

	if err != nil {
		return nil, err
	}

	return buf, nil
}

func write(stream quic.Stream, buf []byte) error {
	n_bytes, err := stream.Write(buf)
	fmt.Println("client: wrote", n_bytes, "bytes")
	if err != nil {
		return err
	}

	return nil
}
